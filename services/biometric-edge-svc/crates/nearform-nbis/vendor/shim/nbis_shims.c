/* NBIS library globals normally provided by the NBIS tool mains.
 *
 * We link the NBIS *libraries* without the tool mains (bin/ is not
 * vendored), so the globals the libraries expect must be defined here:
 * - `verbose_bozorth`: read by bozorth3 lib when printing match traces.
 * - `errorfp`: error stream used by bozorth3 lib diagnostics.
 * - `m1_xyt`: .xyt file format flag (0 = xyt3, upstream tool default).
 * - `min_computable_minutiae`: minimum minutiae for a real score
 *   (upstream tool default = MIN_COMPUTABLE_BOZORTH_MINUTIAE).
 *
 * Defaults mirror the upstream bozorth3 tool main; none affect match
 * scores except min_computable_minutiae's documented floor.
 */
#include <stdio.h>
#include <bozorth.h>

int verbose_bozorth = 0;

FILE *errorfp = NULL;

int m1_xyt = 0;

int min_computable_minutiae = MIN_COMPUTABLE_BOZORTH_MINUTIAE;

/* Route library diagnostics to stderr before any matching runs. */
__attribute__((constructor)) static void eel_nbis_shim_init(void) {
    if (errorfp == NULL) {
        errorfp = stderr;
    }
}
