/* SPDX-License-Identifier: BSD-2-Clause */
/* FS-only diagnostic table for qsee-supplicant. RPMB uses its separate broker.
 * Run with --state-dir /run/fingerprint-state; no Android filesystem mounts. */
#include "qsee_supplicant.h"
const struct qs_service qs_builtin_services[] = {
 {QS_FS_SERVICE_ID, QS_FS_BUFFER_SIZE, qs_fs_dispatch, qs_fs_reset},
 {QS_GPFS_SERVICE_ID, QS_GPFS_BUFFER_SIZE, qs_gpfs_dispatch, NULL},
};
const size_t qs_builtin_service_count=sizeof(qs_builtin_services)/sizeof(qs_builtin_services[0]);
