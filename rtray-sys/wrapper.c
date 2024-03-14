#include "tray/tray.h"

// Static wrappers

void tray_update__extern(struct tray *tray) { tray_update(tray); }
int tray_init__extern(struct tray *tray) { return tray_init(tray); }
int tray_loop__extern(int blocking) { return tray_loop(blocking); }
void tray_exit__extern(void) { tray_exit(); }
