// Adapted from
// https://github.com/tock/libtock-c/blob/c0202f9ab78da4a6e95f136cf5250701e3778f63/libtock/sys.c

#include <errno.h>
#include <sys/stat.h>
#include <sys/types.h>

// XXX Suppress unused parameter warnings for this file as the implementations
// are currently all just stubs
#pragma GCC diagnostic ignored "-Wunused-parameter"

// XXX Suppress missing prototype warnings for this file as the headers should
// be in newlib internals, but first stab at including things didn't quite work
// and the warnings are just noise
#pragma GCC diagnostic ignored "-Wmissing-declarations"
#pragma GCC diagnostic ignored "-Wmissing-prototypes"
#pragma GCC diagnostic ignored "-Wstrict-prototypes"

// XXX Also suppress attribute suggestions as these are stubs
#pragma GCC diagnostic ignored "-Wsuggest-attribute=const"

// ------------------------------
// LIBC SUPPORT STUBS
// ------------------------------

void* __dso_handle = 0;

int _unlink(const char* pathname) {
  return -1;
}

int _isatty(int fd) {
  if (fd == 0) {
    return 1;
  }
  return 0;
}
int _open(const char* path, int flags, ...) {
  return -1;
}
int _close(int fd) {
  return -1;
}
int _fstat(int fd, struct stat* st) {
  st->st_mode = S_IFCHR;
  return 0;
}
int _lseek(int fd, uint32_t offset, int whence) {
  return 0;
}
int _read(int fd, void* buf, uint32_t count) {
  return 0;   // k_read(fd, (uint8_t*) buf, count);
}
void _exit(int __status) {
  // Force a fault:
  *((unsigned int *) NULL) = 0xDEADBEEF;
}
int _getpid(void) {
  return 0;
}
int _kill(pid_t pid, int sig) {
  return -1;
}
caddr_t _sbrk(int incr) {
  /* return (caddr_t) -1; */
  intptr_t nullptr = 0;
  *((size_t *) nullptr) = 0;
}
int _write(__attribute__ ((unused)) int fd, __attribute__ ((unused)) const void* buf, uint32_t count) {
    return count;
}
