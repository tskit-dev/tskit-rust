#include <check.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>

#include "subprojects/kastore/kastore.h"

/* Helper to create a malformed kastore file with a given num_items in header */
static const char *create_malformed_file(const char *path, uint64_t num_items)
{
    FILE *f = fopen(path, "wb");
    if (!f) return NULL;
    /* KAS magic number + version */
    unsigned char header[64];
    memset(header, 0, sizeof(header));
    /* Magic: "\211KAS\r\n\032\n" */
    header[0] = 0x89;
    header[1] = 'K'; header[2] = 'A'; header[3] = 'S';
    header[4] = '\r'; header[5] = '\n'; header[6] = 0x1a; header[7] = '\n';
    /* Version major=1, minor=0, patch=0 (uint16 each) */
    uint16_t ver = 1;
    memcpy(header + 8, &ver, 2);
    /* num_items at offset 16 (uint64) */
    memcpy(header + 16, &num_items, 8);
    /* file_size at offset 24 - set to header size to keep it minimal */
    uint64_t file_size = 64;
    memcpy(header + 24, &file_size, 8);
    fwrite(header, 1, 64, f);
    fclose(f);
    return path;
}

START_TEST(test_num_items_overflow_rejected)
{
    /* Invariant: opening a kastore file with an absurdly large num_items
       must return an error, never succeed with a corrupted allocation */
    uint64_t payloads[] = {
        0xFFFFFFFFFFFFFFULL,   /* Exploit: huge num_items causing overflow */
        (uint64_t)SIZE_MAX / 2, /* Boundary: half of SIZE_MAX */
        2,                      /* Valid small value (will fail due to missing data, but not overflow) */
    };
    int num_payloads = sizeof(payloads) / sizeof(payloads[0]);
    char path[] = "/tmp/kas_test_XXXXXX";

    for (int i = 0; i < num_payloads; i++) {
        int fd = mkstemp(path);
        ck_assert_int_ge(fd, 0);
        close(fd);

        create_malformed_file(path, payloads[i]);

        kastore_t store;
        int ret = kastore_open(&store, path, "r", 0);
        /* For huge num_items, the library MUST reject with an error code.
           It must NOT return success with a corrupted/undersized buffer. */
        if (payloads[i] > 1000000) {
            ck_assert_int_ne(ret, 0);
        }
        if (ret == 0) {
            kastore_close(&store);
        }
        unlink(path);
    }
}
END_TEST

Suite *security_suite(void)
{
    Suite *s;
    TCase *tc_core;

    s = suite_create("Security");
    tc_core = tcase_create("Core");

    tcase_add_test(tc_core, test_num_items_overflow_rejected);
    suite_add_tcase(s, tc_core);

    return s;
}

int main(void)
{
    int number_failed;
    Suite *s;
    SRunner *sr;

    s = security_suite();
    sr = srunner_create(s);

    srunner_run_all(sr, CK_NORMAL);
    number_failed = srunner_ntests_failed(sr);
    srunner_free(sr);

    return (number_failed == 0) ? EXIT_SUCCESS : EXIT_FAILURE;
}