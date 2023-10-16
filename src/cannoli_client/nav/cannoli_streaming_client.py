import socket, time, os, signal
import subprocess, pathlib, json

binaries = pathlib.Path(pathlib.Path(__file__).parent.parent.resolve()) / "binaries"
data = pathlib.Path(pathlib.Path(__file__).parent.parent.resolve()) / "data"
BUFFER_SIZE = 1024

QEMU_SENTINEL_VALUE = '>'

POISON = 0x36afb081;
INIT_EVENT_COUNT = 1 
SCHEMAS = ["heap"]


# TODO: create a universal timeout

class CannoliStreamingClient:

    def __init__(self, exec_name):
        """
        initializes qemu and cannoli socket connections
        """
        self.recv_buf: bytes = b''
        self.pid = os.getpid()
        self.recv_path_cannoli = "/tmp/nav_" + str(self.pid)
        print("[*] Spawning QEMU process ")
        target_path = "/opt/leetasm/examples/" + f"{exec_name}"
        self.qemu = subprocess.Popen([
            "../../qemu/build/qemu-mipsel",
            "-cannoli",
            "/opt/leetasm/cannoli_client/target/debug/libcannoli_client.so",
            target_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE)
        self.recv_path_qemu = "/tmp/nav_" + str(self.qemu.pid)

        # create receive socket for Cannoli feedback
        # must do this first so that Cannoli can initialize init_pid()

        try:
            os.unlink(self.recv_path_cannoli)
        except OSError:
            if os.path.exists(self.recv_path_cannoli):
                raise

        self.cannoli_sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.cannoli_sock.bind(self.recv_path_cannoli)
        self.cannoli_sock.listen(1)
        # print("[*] NAV: listening on " + self.recv_path_cannoli)
        self.conn, addr = self.cannoli_sock.accept()
        # print("[*] NAV: received connection on " + self.recv_path_cannoli)
        self.conn.settimeout(6)

        # create socket to listen for shell process spawned with win()
        # will be spawned with ppid == qemu.pid
        self.qemu_sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.qemu_sock.bind(self.recv_path_qemu)
        self.qemu_sock.listen(1)

        # flush all init (pre-main) events and return the last one,
        # representative of the "initial state"
        self._flush()

    def _flush(self) -> str:
        for _ in range(INIT_EVENT_COUNT ):
            self.try_read()
            # print(f"Init {self.try_read()}")
    
    def try_write(self, action) -> None:
        """
        invoke executable with input string
        action: input string
        """
        n = self.qemu.stdin.write(action.encode() + b'\n')
        # print(f"[*] NAV: sent {n} bytes: {action.encode()} to qemu process")
        self.qemu.stdin.flush()

    # cannoli_client packet format
    # POISON value as a u32 integer (4 bytes) followed by the schema length
    # for every possible schema (this must be synchronized b/w cannoli and
    # the receiving client). All lengths are little endian.
    #
    # schema lengths will be null for any event type that was not generated
    # by the current event.
    #
    # The current version of cannoli only generates 1 schema, so there is
    # only 1 length value and one data blob following the header.
    #
    #       u32            u32             u32       ...       u32
    # .-------------.---------------.--------------.-...-.--------------.
    # |    POISON   |  Schema 1 len | Schema 2 len | ... | Schema n len | ...
    # `-------------`---------------`--------------`-...-`--------------`
    #
    #       Schema x len    Schema y len
    #     .---------------.---------------.-----...------.
    # ... | Schema x data | Schema y data |     ...      |
    #     `---------------`---------------`-----...------`

    # renaming "try_read" per API screenshot
    def try_read(self) -> str:  # TO DO: update to schema return
        # try: # TO DO: holding off on this try right now because it's difficult
        # to debug errors when handling with an except statement
        self.recv_buf += self.conn.recv(BUFFER_SIZE)
        poison = int.from_bytes(self.recv_buf[:4], "little")
        if POISON != poison:
            # packet corrupted, flush until next poison is found
            self.reset_buffer()
            return None
        self.recv_buf = self.recv_buf[4:]

        # get schema lengths. Indexes correspond to the schemas defined in
        # SCHEMAS
        schema_lengths = []
        for i in range(len(SCHEMAS)):
            schema_lengths.append(int.from_bytes(self.recv_buf[:4], "little"))
            self.recv_buf = self.recv_buf[4:]

        schema_data = {}
        # use schema lengths to parse schema data (if available)
        for i in range(len(SCHEMAS)):
            if len(self.recv_buf) < schema_lengths[i]:
                # packet could be fragmented, try and recv more
                try:
                    while len(self.recv_buf) < schema_lengths[i]:
                        extra = self.conn.recv(BUFFER_SIZE)
                        if extra == b'':
                            # no more data remaining, weird state, flush buffer
                            self.reset_buffer()
                            return None
                        else:
                            self.recv_buf += extra
                except socket.timeout:
                    # no more data remaining, weird state, flush bluffer
                    self.reset_buffer()
                    return None
                except Exception as e:
                    print("Unhandled execption while recieving fragmented " \
                          + "packet: ", e)

            # full event available in buffer. Parse
            data = self.recv_buf[:schema_lengths[i]]
            self.recv_buf = self.recv_buf[schema_lengths[i]:]
            try:
                # TO DO: can there be more than one schema returned??
                # schema_data[SCHEMAS[i]] = json.loads(data.decode())
                return json.loads(data.decode())
            except:
                # failure to deserialize, scrap rest of buffer until next
                # poison
                self.reset_buffer()
                return None

            # TO DO: if returning more than one schema, would accumulate in
            # dict or list in `try` clause above and return here
            # return ??

        # the following exceptions return none:
        # socket.timeout, int conversion failure, buffer out of space
        # except:
        return None

    # finds next poison value to reset buffer. If dne, null buffer out
    def reset_buffer(self) -> None:
        idx = self.recv_buf.find(POISON.to_bytes(4, "little"))
        if (idx >= 0):
            self.recv_buf = self.recv_buffer[idx:]
        else:
            self.recv_buffer = b''

    # def try_read(self) -> str: # TO DO: make this return the schema
    #     """
    #     generate analysis and return response
    #     """
    #     return self._try_read()
    # returns = []
    # buf: str = ''
    # while True:
    #     try:
    #         length: int = 0
    #         buf += self.conn.recv(BUFFER_SIZE).decode()
    #         while True:
    #             length = int(buf[:8], 16)
    #             if length > len(buf): break
    #             returns.append(buf[8:8+length].strip())
    #             buf = buf[8+length:]
    #             if buf == '': break
    #         if buf == '': break
    #     except socket.timeout:
    #         break
    #     except Exception as e:
    #         print("caught unhandled exception", e)
    #         exit(1)

    # # return self.conn.recv(BUFFER_SIZE).decode()
    # return returns

    def reset(self):
        """
        kills socket connections
        """
        self.qemu.kill()
        self.conn.close()
        self.cannoli_sock.close()
        # signal.signal(signal.SIGINT, self._handle_kill)

    def _handle_kill(self):
        if self.qemu and self.qemu.poll():
            self.qemu.terminate()
