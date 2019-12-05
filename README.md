# Fasta file upload and sequence search

### Run

``` open web browser to localhost:3000 and upload file(s) then ```
``` open web browser to localhost:3000/search and input search sequence ```

### Result

``` file(s) will show up in ./tmp in the same directory as the running process ```
``` search will search all fasta files in ./tmp for the sequence and print out the number of occurrences in each chromosome in the console ```

Note: this is a naive implementation and will panic on any error
