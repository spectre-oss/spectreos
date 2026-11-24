/* SPDX-License-Identifier: GPL-2.0-only */
/* One QSEECOM receiver owns RPMB, FS and GPFS. See diagnostics README. */
#define main standalone_rpmb_main
#include "rpmb-bridge.c"
#undef main
#include "qsee_supplicant.h"
static int storage_rpmb(struct qs_store *store,void *buffer,size_t size) {
 (void)store;
 if(size!=BUFSZ)return -1;
 if(rpmb.fd<0 && rpmb_ufs_init())return -1;
 return dispatch(buffer);
}
const struct qs_service qs_builtin_services[] = {
 {8192,BUFSZ,storage_rpmb,NULL},
 {QS_FS_SERVICE_ID,QS_FS_BUFFER_SIZE,qs_fs_dispatch,qs_fs_reset},
 {QS_GPFS_SERVICE_ID,QS_GPFS_BUFFER_SIZE,qs_gpfs_dispatch,NULL},
};
const size_t qs_builtin_service_count=sizeof(qs_builtin_services)/sizeof(qs_builtin_services[0]);
