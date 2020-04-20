//
//  Use this file to import your target's public headers that you would like to expose to Swift.
//
#pragma once

#include <stdint.h>

typedef struct md_notes_runtime md_notes_runtime;

md_notes_runtime* md_notes_runtime_new(void);

void md_notes_runtime_free(md_notes_runtime*);

uint16_t md_notes_runtime_server_port(md_notes_runtime*);

uint8_t md_notes_runtime_open_notes(md_notes_runtime*, const char *);

void md_notes_runtime_close_notes(md_notes_runtime*, uint8_t);
