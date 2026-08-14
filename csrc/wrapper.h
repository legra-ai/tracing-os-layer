#include <os/log.h>
#include <os/activity.h>
#include <mach-o/loader.h>

void tracing_os_layer_log_with_type(os_log_t log, os_log_type_t type, const char *message);
os_log_t tracing_os_layer_log_default(void);
os_activity_t tracing_os_layer_activity_create(
    const char *description,
    os_activity_t parent,
    os_activity_flag_t flags
);
os_activity_t tracing_os_layer_activity_current(void);

