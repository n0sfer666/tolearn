#include <stdlib.h>
#include <string.h>

#include "ggml.h"
#include "whisper.h"

struct tolearn_speech {
    struct whisper_context * ctx;
};

static void quiet(enum ggml_log_level level, const char * text, void * user_data) {
    (void) level;
    (void) text;
    (void) user_data;
}

void tolearn_speech_hush(void) {
    whisper_log_set(quiet, NULL);
    ggml_log_set(quiet, NULL);
}

struct tolearn_speech * tolearn_speech_open(const char * model, int gpu) {
    struct whisper_context_params params = whisper_context_default_params();
    params.use_gpu = gpu != 0;

    struct whisper_context * ctx = whisper_init_from_file_with_params(model, params);
    if (ctx == NULL) {
        return NULL;
    }
    struct tolearn_speech * self = malloc(sizeof(struct tolearn_speech));
    if (self == NULL) {
        whisper_free(ctx);
        return NULL;
    }
    self->ctx = ctx;
    return self;
}

void tolearn_speech_close(struct tolearn_speech * self) {
    if (self == NULL) {
        return;
    }
    whisper_free(self->ctx);
    free(self);
}

int tolearn_speech_hear(
    struct tolearn_speech * self,
    const float * pcm,
    int samples,
    const char * language,
    int threads,
    char ** heard
) {
    struct whisper_full_params params = whisper_full_default_params(WHISPER_SAMPLING_GREEDY);
    params.print_progress = false;
    params.print_realtime = false;
    params.print_timestamps = false;
    params.print_special = false;
    params.translate = false;
    params.no_timestamps = true;
    params.language = language;
    params.detect_language = false;
    params.n_threads = threads;

    int failed = whisper_full(self->ctx, params, pcm, samples);
    if (failed != 0) {
        return failed;
    }

    int segments = whisper_full_n_segments(self->ctx);
    size_t room = 1;
    for (int at = 0; at < segments; at++) {
        room += strlen(whisper_full_get_segment_text(self->ctx, at));
    }
    char * text = malloc(room);
    if (text == NULL) {
        return -1;
    }
    text[0] = '\0';
    for (int at = 0; at < segments; at++) {
        strcat(text, whisper_full_get_segment_text(self->ctx, at));
    }
    *heard = text;
    return 0;
}

void tolearn_speech_forget(char * heard) {
    free(heard);
}
