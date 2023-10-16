#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>
#include <unistd.h>

int g_Y = 0; /* specifically uninitialized */
int g_Z = 0xDEADBEEF;

int flip1or2(/* int g_Y */)
{
  switch(g_Y)
    {
    case 0:
      g_Y = 1;
      break;
    case 1:
      g_Y = 2;
      break;
    case 2:
      g_Y = 1;
      break;
    default:
      g_Y = 1;
    }
  
  return g_Y;
}

/**
 * Emulate a "read" operation from "A"'s address space 
 * AxRake, here we come!
 */
int readXFromA()
{
  int Ax = 0;
  srandom(time(NULL));
  Ax = ((int)random() % 8 + 1);
  return Ax;
}

/** 
 * Alternative refined definition from actual
 * implementation semantics
 */
int flip1or2_alt(/* int B.g_Y, int A.g_X, int B.g_Z */)
{

  //modify g_Y by reading A.g_X's value
  //add a case stmt if A>=7

  /** This would be a normal thing to do, missing in default impl.*/
  if (g_Y<0)
    g_Y = 0;

  /** And here is the composed linkage, dev. is getting ahead of themselves*/
  //read(A.g_X), if >= 7, then g_Y = A.g_X

  g_Z = readXFromA();
  if(g_Z >= 7)
  {
    g_Y = g_Z;
  }
  
  switch(g_Y)
    {
    case 0:
      g_Y = 1;
      break;
    case 1:
      g_Y = 2;
      break;
    case 2:
      g_Y = 1;
      break;
    case 7:
      g_Y = g_Z;
      break;
    default:
      g_Y = 1;
    }
  
  return g_Y;
}


int main(int argc,
	 char* argv[])
{
  int ret = 0;  
  fprintf(stdout, "This is flip1or2() driver.\n");

  fprintf(stdout, "\tB.y=%d\n", g_Y);
  flip1or2();
  fprintf(stdout, "\tB.y=%d\n", g_Y);
  flip1or2();
  fprintf(stdout, "\tB.y=%d\n", g_Y);
  flip1or2();
  fprintf(stdout, "\tB.y=%d\n", g_Y);
  flip1or2();
  fprintf(stdout, "\tB.y=%d\n", g_Y);
  flip1or2();
  fprintf(stdout, "\tB.y=%d\n", g_Y);
  flip1or2();
  fprintf(stdout, "\tB.y=%d\n", g_Y);
  
  ret = 0;  
  return ret;
}
