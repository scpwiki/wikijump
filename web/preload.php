<?php

/**
 * Preloading libraries which cannot occur at runtime.
 *
 * This is most notable for FFI, as we have "ffi.enabled" set to "preload".
 */

FFI::load('/usr/local/include/ftml.h');
opcache_compile_file(__DIR__ . 'web/php/Wikitest/FTML/FtmlRaw.php');
