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
    int c = short_circuit_and();
    return c;
}

int loop() {
    int i = 0;
    while (i < 10) {
        i = i + 1;
    }
    return i;
}

int caller(){
    called(2,2);
    return called(1,2);
}

int called(int a,int b){
    return a  +b;
}

static string name(){
    return "test";
}

static string var_string(){
    string name = "name";
    return ret_string(name);
}

string ret_string(string name){
    return name;
}

int get_module(string name) {
    int handle = SysCall(0x0, 0x10, 1, name);
    if (handle == 0) {
        SysCall(0x0, 0x3, "slFindModule: module not found", name);
        SysCall(0x0, 0x1, "ERROR: slFindModule\n");
    }
    return handle;
}

static void get_global_manager(){
    int gm = get_module("GlobalManager");
    return;
}


static int unlock_pokemon(int pokemon_objectId)

{
  int iVar1;
  int iVar2;

  iVar1 = get_module("DisposManager");
  iVar2 = get_module("GlobalManager");
  iVar1 = SysCall(0,0x15,4,iVar1,pokemon_objectId);
  if (iVar1 == -2) {
    return 0;
  }
  if (iVar1 == -1) {
    return 0;
  }
  SysCall(0,0x15,0x29,iVar2,iVar1);
  SysCall(0,0x15,0x28,iVar2,iVar1);
  return 1;
}

int FUN_000502a0()

{
  int iVar1;
  int uVar2;
  int iVar3;

  iVar1 = get_module("ObjectManager");
  uVar2 = SysCall(0,0x15,0,iVar1,0,0);
  uVar2 = SysCall(0,0x15,1,iVar1,uVar2);
  iVar3 = SysCall(0,0x15,0x17,uVar2,0);
  if (iVar3 != -1) {
    uVar2 = SysCall(0,0x15,1,iVar1,iVar3);
    iVar1 = SysCall(0,0x15,0x18,uVar2);
    if (iVar1 == 6) {
      iVar1 = SysCall(0,0x15,0x19,uVar2);
      return iVar1;
    }
  }
  return -1;
}

int FUN_00050f20(int param_1,int param_2,int param_3)

{
  int iVar1;
  int iVar2;
  int uVar3;
  int uStack_c;

  iVar1 = get_module("ObjectManager");
  iVar2 = get_module("EventScript");
  if (param_3 == 1) {
    uStack_c = param_2;
  }
  else {
    uStack_c = SysCall(0,0x15,1,iVar2,param_1);
  }
  uVar3 = SysCall(0,0x15,4,iVar1,uStack_c);
  iVar1 = SysCall(0,0x15,1,iVar1,uVar3);
  return iVar1;
}

void wait(){
    Pause(1);
    Pause(2);
    return;
}

void OPEN(int param_1)

{
  int uVar1;
  int uVar2;

  uVar1 = SysCall(0,0x15,4,param_1);
  SysCall(0,0x15,0,uVar1,param_1,2);
  uVar2 = SysCall(0,0x15,0x1e,param_1);
  SysCall(0,0x15,4,uVar2,1,0);
  SysCall(0,0x15,4,uVar2,1,1);
  return;
}


static void CLOSE(int param_1)

{
  int iVar1;
  int uVar2;

  iVar1 = get_module("ObjectManager");
  uVar2 = SysCall(0,0x15,0,param_1);
  uVar2 = SysCall(0,0x15,1,iVar1,uVar2);
  iVar1 = get_module("GlobalManager");
  iVar1 = SysCall(0,0x15,1,iVar1,"GF0002");
  if (iVar1 == 1) {
    SysCall(0,0x15,0,uVar2,2);
  }
  else {
    SysCall(0,0x15,4,param_1);
    SysCall(0,0x15,1,uVar2,0);
  }
  return;
}