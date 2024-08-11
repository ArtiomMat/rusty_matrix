// Auto-adjusting Matrix screen for the terminal, priniting in actual matrix codes o.O, nah just Japanese.

#include <sys/ioctl.h>
#include <unistd.h>

#include <uchar.h> // For the "matrix" chars

#include <stdio.h>

static const unsigned SLEEP_TIME = 1000 * 250; // After the *, it's the time in ms.

static short w, h;

static const char* glyphs[] = {
	"\xE3\x81\x82",
	"\xE3\x81\x83",
	"\xE3\x81\x84",
	"\xE3\x81\x85",
	"\xE3\x81\x86",
	"\xE3\x81\x87",
	"\xE3\x81\x88",
	"\xE3\x81\x89",
	"\xE3\x81\x8A",
	"\xE3\x81\x8B",
	"\xE3\x81\x8C",
	"\xE3\x81\x8D",
};

static void init() {
	struct winsize ws;
	ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws);

	w = ws.ws_col;
	h = ws.ws_row;
}

static void run() {
	
}

int main() {
	init();
	puts("\xE3\x81\x82\n");
	while (1) {
		run();
		usleep(SLEEP_TIME);
	}

	return 0;
}
