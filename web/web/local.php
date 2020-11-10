<?php
require ('../php/setup.php');

try {

    $controller = new UploadedFileFlowController();
    $out = $controller->process();

} catch (Exception $e) {
    echo "A nasty error has occurred. If the problem repeats, please fill (if possible) a bug report.";
    echo "<br/><br/>";
    echo $e;
}
