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

int check(int id) {
    if (id == 0) {
        return 1;
    }
    return 0;
}

int else_cases() {
    if (1 == 1) {
        int x = 0;
    }
    else{
        int y = 1;
    }

    return 0;
}

int multi_decl() {
    int x;
    x = 1;
    int y = 2;
    if (1 == 1) {
        int x = 0;
    }
    else{
        int y = 1;
    }

    return 0;
}

int chained_ahritmetic(bool enrich){
    int x = 1 + 2 + 3 + 4 + 5;
    if (enrich){
        x = x +1;
    }
    int y = 1+2;
    return x+y;
}

int multi_ifs(int case){
    int x;
    if(1 == case){
        x = 1;
    }
    else{
       if(2==case){
        x =2;
        }
        else{
        x = 0;
        }
    }
    return x;
}

int dynamic_cond(int x) {
    if (x) {
        return 1;
    }
    return 0;
}

int empty_blocks() {
    if (1) {
    } else {
    }
    return 0;
}

int deep_shadow() {
    int x = 1;
    if (1) {
        int x = 2;
        if (1) {
            int x = 3;
        }
    }
    return x; // should be 1
}

int short_circuit_and(){
    int x = 0;
    if (false && true) {
        x = 1;
    }
    return x;
}
int short_circuit_or(){
    int x = 0;
    if (false || true) {
        x = 1;
    }
    return x;
}

int loop() {
    int i = 0;
    while (i < 10) {
        i = i + 1;
    }
    return i;
}