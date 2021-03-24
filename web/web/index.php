<?php

use Ozone\Framework\Exceptions\OzoneDatabaseException;
use Ozone\Framework\OzoneLogger;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WikiFlowController;

require ('../php/setup.php');

// to avoid caching
header("Cache-Control: no-cache, must-revalidate"); // HTTP/1.1
header("Expires: Mon, 26 Jul 1997 05:00:00 GMT"); // Date in the past

try {
    // set anti-session-riding token
    setsecurecookie("wikijump_token7", md5(rand(0, 10000)), 0, '/', GlobalProperties::$SESSION_COOKIE_DOMAIN);
    // If this is coming from a custom domain, set a token7 so they can work with the admin panel if needed.
    if($_SERVER['HTTP_HOST'] != GlobalProperties::$URL_HOST) {
        setsecurecookie("wikijump_token7", md5(rand(0, 10000)), 0, '/', '.'.$_SERVER['HTTP_HOST']);
    }
    $controller = new WikiFlowController();
    $out = $controller->process();
} catch (OzoneDatabaseException $e) {
	echo "<html><head><title>Database Error</title></head><body>";
	echo "<h1>Database error</h1>";
	echo "<p>Make sure PostgreSQL server is running and accepts connection for credentials stored in " . WIKIJUMP_ROOT . "/conf/wikijump.ini</p>";
	echo "<p>If you haven't configured Wikijump database yet, consult the INSTALL file.</p>";
	echo "<hr/>";
	echo "<p>Below is the original error:<br/>{$e->getMessage()}</p>";
	echo "</body></html>";
} catch (Exception $e) {
    echo "A nasty error has occurred. If the problem repeats, please fill (if possible) a bug report.";
    echo "<br/><br/>";
    echo $e;
    // hope the logger is initialized...
    $logger = OzoneLogger::instance();
    $logger->error("Exception caught:\n\n" . $e->__toString());
}
