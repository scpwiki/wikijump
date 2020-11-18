<?php
require ('../php/setup.php');

$fileUri = $_SERVER['QUERY_STRING'];
$content = file_get_contents($fileUri);

// replace links to https://www.wikijump.com/... and to the current host.
# XXX mix-up between http and https?
$content = preg_replace(';url\(http://([a-z0-9\-]+)\.{$URL_DOMAIN_PREG}/;i', 'url(https://\\1.{$URL_DOMAIN}/', $content);

header("Content-Type: text/css");
echo $content;
