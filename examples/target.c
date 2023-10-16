#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>
#include <unistd.h>

/**
 * This is a global variable representing the 'state' of this target
 * program. We invent an arbitrary specification that claims that
 * when `shared_global_var`'s value is above 500, it is in State A
 * and when it is below 500 it is State B and if it is exactly 500 it
 * is State C.
 */
int gvar = 1000;

int gvar1 = 5000;

int a = 680;
int b = 785;

int *memory = 0x0;

void doB()
{
  int x = 222;
  gvar = gvar / 2 + 3;
  gvar1 = gvar1 + x;
  x = x - 10;
}

void doA(int x)
{
  gvar -= x;
}

int reviseAB()
{
  int ret = 0;
  a = a - 2;
  b = b * 10;

  ret = a + b;
  return ret;
}

int main(int argc,
	 char* argv[])
{
  int i = 99;
  int ret = 0;
  
  fprintf(stdout, "This is the target program.\n");
  fprintf(stdout, "\targc=%d\n", argc);
  fprintf(stdout, "\tgvar=%d\n", gvar);
    
  doA(i);
  fprintf(stdout, "\tgvar=%d\n", gvar);
  i=i-56;
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  doB();
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  doA(i);
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  doA(i);
  fprintf(stdout, "\tgvar=%d\n", gvar);

  b = b + 2;
  a = a + 300;
  
  memory = malloc(1000);
  if (NULL==memory)
    return -1;

  *memory = 0x41414141;

  
  i = i - 90;
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  doB();
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  doA(i);
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  gvar = 0;
  *memory = 0x90909090;
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  gvar = 500;
  fprintf(stdout, "\tgvar=%d\n", gvar);  
  gvar = 1000;
  fprintf(stdout, "\tgvar=%d\n", gvar);

  ret = reviseAB();
  
  free(memory);
  memory = NULL;

  ret = 0;
  
  return ret;
}
