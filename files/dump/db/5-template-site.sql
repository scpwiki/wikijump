
INSERT INTO site VALUES (2, 'Plain Wiki', 'powered by wikidot software', 'template-plain', '', 'en', NULL, NULL, true, 'start', false, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO site_settings VALUES (2, false, false, '', 314572800, false, 'system:join', 50, 20, true, NULL, false, false, 10485760, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO site_super_settings VALUES (2, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO admin VALUES (2, 2, 1, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO member VALUES (2, 2, 1, NOW());;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (11,
    2, '_default',
    false, 20,
    false, 'e:rm;c:rm;m:rm;d:rm;a:rm;r:rm;z:rm;o:rm',
    true, 1, NULL,
    false, 'nav:top', 'nav:side',
    NULL, false, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (12,
    2, 'system',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (13,
    2, 'search',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (14,
    2, 'admin',
    false, 21,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    false, 'nav:top', NULL,
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

