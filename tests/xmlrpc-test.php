<?php
require_once('../php/setup.php');
require_once('../php/pingback/PingBack.php');

$pb = new PingBack("http://download.fedoraproject.org/pub/fedora/linux/releases/9/Fedora/x86_64/iso/Fedora-9-x86_64-DVD.iso", "http://quake.wikidot.com/123");
$p = $pb->ping();
