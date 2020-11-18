<?php
/**
 * This class is responsible for handling exceptions which are thrown
 * when processing modules/screens.
 */
class ProcessExceptionHandler
{

    public function handleInlineModule($exception, $runData)
    {
        // rollback the transaction
        $db = Database::connection();
        $db->rollback();
        $out.= '<div class="error-block">';
        if ($exception instanceof ProcessException) {
            $out.=nl2br($exception->getMessage());
        } elseif ($exception instanceof WDPermissionException) {
            $out.='<div class="title">Permission error</div>';
            $out.=nl2br($exception->getMessage());
        } else {
            $out.="An error occured when processing your request.";
            // LOG ERROR TOO!!!
            $logger = OzoneLogger::instance();
            $logger->error("Exception caught while processing inline module:\n\n".$exception->__toString());
        }
        $out.='</div>';
        return $out;
    }

    public function handleAjaxRequest($exception, $runData)
    {
        $db = Database::connection();
        $db->rollback();
        if ($exception instanceof ProcessException) {
            $runData->ajaxResponseAdd("message", $exception->getMessage());
            $runData->ajaxResponseAdd("status", $exception->getStatus());
        } elseif ($exception instanceof WDPermissionException) {
        } else {
            $runData->ajaxResponseAdd("message", "An error occured when processing your request.");
            $runData->ajaxResponseAdd("status", "not_ok");
        }
    }
}
