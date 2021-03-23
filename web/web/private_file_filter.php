<?php

use Ozone\Framework\OzoneLogger;

require ('../php/setup.php');

try {

    $controller = new PrivateFileFlowController();
    $out = $controller->process();
} catch (Exception $e) {
    echo "A nasty error has occurred. If the problem repeats, please fill (if possible) a bug report.";
    echo "<br/><br/>";
    echo $e;
    // hope the logger is initialized...
    $logger = OzoneLogger::instance();
    $logger->error("Exception caught:\n\n" . $e->__toString());
}
