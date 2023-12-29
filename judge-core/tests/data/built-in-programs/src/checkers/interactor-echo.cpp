#include "testlib.h"
#include <iostream>

using namespace std;

int main(int argc, char *argv[]) {
    setName("Echo interactor");
    registerInteraction(argc, argv);

    // reads string from test (input) file
    string x = inf.readString();
    // write to stdout
    cout << x << endl;
    // write to output file
    tout << ouf.readString() <<endl;

    // just message
    quitf(_ok, "interactor exited");
}
