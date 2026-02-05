#include <stdio.h>
#include <stdlib.h>
#include <sys/random.h>
#include <string.h>
#include <unistd.h>
#include <linux/limits.h>


// Read an input file and xor with random data, writing xored 
// data to 01 and random data to 02
// Users can xor original random data with xored data to recover original file
// xored = <input file> ^ /dev/urandom (each byte used written to 02)
// <input file> = xored ^ original random data (02)
int main(int argc, char *argv[]) {

    // parse args for -i <input file> -o1 <output path to part 1> -o2 <outpath to part 2> 
    int flags, opt;
    int tfnd;
    char inputPath[PATH_MAX]; 
    char outputPath[PATH_MAX];
    char padPath[PATH_MAX];


    memset(inputPath, 0, sizeof(inputPath));
    memset(outputPath, 0, sizeof(outputPath));
    memset(padPath, 0, sizeof(padPath));

    tfnd = 0;
    flags = 0;
    while ((opt = getopt(argc, argv, "i:o:p:")) != -1) {
        switch (opt) {
            case 'i':
                strlcpy(inputPath, optarg, sizeof(inputPath));
                break;
            case 'o':
                strlcpy(outputPath, optarg, sizeof(outputPath));
                break;
            case 'p':
                strlcpy(padPath, optarg, sizeof(padPath));
                break;
            default: /* '?' */
                fprintf(stderr, "Usage: %s -i <input path> -o <output path> -p <pad output path>\n",
                argv[0]);
                exit(EXIT_FAILURE);
        }
    }

    printf("inputPath %s\noutputPath %s\npadPath %s\n", inputPath, outputPath, padPath);

    /* Other code omitted */

    // open <input file> for reading
    // open output files
    // open /dev/urandom, read in size_t bytes, write to 02 then fsync
    // open 02 for reading
    // xor 02[offset] with <input file>[offset] from 0 to eof of <input file>
    exit(EXIT_SUCCESS);
}
