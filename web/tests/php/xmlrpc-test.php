<?php

use Wikidot\Pingback\Pingback;

require_once('../php/setup.php');

$pb = new Pingback("http://download.fedoraproject.org/pub/fedora/linux/releases/9/Fedora/x86_64/iso/Fedora-9-x86_64-DVD.iso", "http://test.wikijump.com/123");
$p = $pb->ping();
