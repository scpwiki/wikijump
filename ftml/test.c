#include <stdio.h>

#include "target/debug/build/ftml-38ec7c9a86770ac7/out/ftml.h"

int main(int argc, char **argv)
{
	const char *input1;
	char input2[30];
	char *output;

	printf("%% %s\n", ftml_version());

	input1 = "<< Test >>\n\n \nA \\\nB\n";
	snprintf(input2, sizeof(input2), "\n\tI love my `cat'! %d\n\n", 10);

	output = ftml_preprocess(input1);
	printf("Input  1: <{ %s }>\n", input1);
	printf("Output 1: <{ %s }>\n", output);
	ftml_free(output);

	output = ftml_preprocess(input2);
	printf("Input  2: <{ %s }>\n", input2);
	printf("Output 2: <{ %s }>\n", output);
	ftml_free(output);

	return 0;
}
