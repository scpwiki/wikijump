<?php
declare(strict_types=1);

use Illuminate\Support\Facades\Log;
use Ozone\Framework\Exceptions\OzoneDatabaseException;
use Wikidot\Utils\WikiFlowController;

// to avoid caching
header('Cache-Control: no-cache, must-revalidate'); // HTTP/1.1
header('Expires: Mon, 26 Jul 1997 05:00:00 GMT'); // Date in the past

try {
    $controller = new WikiFlowController();
    $out = $controller->process();
} catch (OzoneDatabaseException $e) {
	echo '<html lang="en"><head><title>Database Error</title></head><body>';
	echo '<h1>Database error</h1>';
	echo '<p>Make sure PostgreSQL server is running and accepts connection for credentials stored in ' . WIKIJUMP_ROOT . '/conf/wikijump.ini</p>';
	echo '<p>If you haven\'t configured Wikijump database yet, consult the INSTALL file.</p>';
	echo '<hr/>';
	echo '<p>Below is the original error:<br/>' . $e->getMessage() . '</p>';
	echo '</body></html>';
} catch (Exception $e) {
    echo 'A nasty error has occurred. If the problem repeats, please fill (if possible) a bug report.';
    echo '<br/><br/>';
    echo $e;
    Log::error("[OZONE] Exception caught:\n\n" . $e->__toString());
}
