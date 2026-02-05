#include<stdio.h>
#include<stdlib.h>
#include <sys/random.h>
#include <string.h>


// Demonstrate 'splitting' plaintext into two random data segments that when 
// XORd with eachother, revealing the plaintext
int main(int argc, char *argv[]) {
	char *plainText = "Hello World!";
	ssize_t buflen = strlen(plainText) + 1;
	char *xorKey = (char *)calloc(1, buflen);
	char *cypherText = (char *)calloc(1, buflen);
	char *extractedPlainText = (char *)calloc(1, buflen);

	if (xorKey != NULL && cypherText != NULL && extractedPlainText != NULL)
	{
		ssize_t xorKeyValLen = getrandom(xorKey, buflen, 0);
		if (xorKeyValLen < buflen)
		{
			fprintf(stderr, "Error in generating xor key\n");
			return -2;
		}

		for (int i = 0; i < buflen; i++)	
		{
			cypherText[i] = plainText[i] ^ xorKey[i];
			fprintf(stdout, "%02x ", (char)cypherText[i] & 0xFF);
		}
		
		fprintf(stdout, "\n");

		for (int i = 0; i < buflen; i++)
		{
			extractedPlainText[i] = cypherText[i] ^ xorKey[i];
			fprintf(stdout, "%02x ", (char)xorKey[i] & 0xFF);
		}

		fprintf(stdout, "\n");

		for (int i = 0; i < buflen; i++)
		{
			fprintf(stdout, "%02x ", (char)plainText[i] & 0xFF);
		}

		fprintf(stdout, "\n\n");

		fprintf(stdout, "Original %s\nExtracted %s\n", plainText, extractedPlainText);
		return 0;
	}


	// If we get here, an error has occurred
	return -1;
}
