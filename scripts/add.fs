static int add(int a, int b) {
    int c = 1;
    int d = 1;
    c = c + d;
    return a + b + c;
}

int simple_math(){
    int a = 1;
    int b = 2;
    int c = 3;

    return a + b + c;
}

int sub(int a, int b) {
    return a - b;
}

int mul(int a, int b) {
    return a * b;
}

int div(int a, int b) {
    return a / b;
}

void no_return() {
    return;
}

void pure_assign(){
    int a;
    a = 2;
    return;
}

void bool_assign() {
    bool a = true;
    bool b = false;
    return;
}

void cmp(){
    bool a = 1 == 1;
    bool b = 1 < 2;
    bool c = 1 > 2;
    bool d = 1 >= 1;
    bool e = 1 <= 2;
    return;
}

bool ret_and(){
    bool a = 1==1;
    bool b = 2 == 2;
    return a && b;
}

bool ret_or(){
    bool a = 1==1;
    bool b = 2 == 2;
    return a || b;
}

bool ret_unary(){
    bool a = true;
    return !a;
}

int ret_int_unary(){
    return -1;
}