#include <time.h>
#include <stdio.h>

int main() {
    clock_t start, step;
    start = clock();
    step = clock();
    while (true) {
        if (clock() - step > 1000000) {
            printf("%lds has passed..\n", (clock() - start) / 1000000);
            step = clock();
        }
    }
}
