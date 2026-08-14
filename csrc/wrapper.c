#include "wrapper.h"

/* Provided by the linker for this shared object. */
extern struct mach_header __dso_handle;

/* The underlying function behind the os_activity_create macro. */
extern os_activity_t _os_activity_create(
    void *dso,
    const char *description,
    os_activity_t activity,
    os_activity_flag_t flags
);

/* The global representing the current activity. */
extern struct os_activity_s _os_activity_current;

void tracing_os_layer_log_with_type(os_log_t log, os_log_type_t type, const char *message) {
    os_log_with_type(log, type, "%{public}s", message);
}

os_log_t tracing_os_layer_log_default(void) {
    return OS_LOG_DEFAULT;
}

os_activity_t tracing_os_layer_activity_create(
    const char *description,
    os_activity_t parent,
    os_activity_flag_t flags
) {
    return _os_activity_create(&__dso_handle, description, parent, flags);
}

os_activity_t tracing_os_layer_activity_current(void) {
    return &_os_activity_current;
}

