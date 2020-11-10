<?php
require ('../php/setup.php');

// to avoid caching
header("Cache-Control: no-cache, must-revalidate"); // HTTP/1.1
header("Expires: Mon, 26 Jul 1997 05:00:00 GMT"); // Date in the past
header("content-type: text/xml");

try {
    $controller = new FeedFlowController();
    $out = $controller->process();
} catch (Exception $e) {
    echo "A nasty error has occurred. If the problem repeats, please fill (if possible) a bug report.";
    echo "<br/><br/>";
    echo $e;
    // hope the logger is initialized...
    $logger = OzoneLogger::instance();
    $logger->error("Exception caught:\n\n" . $e->__toString());
}

