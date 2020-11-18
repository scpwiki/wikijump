<?php
use DB\SitePeer;

class CustomDomainScript extends SmartyScreen
{

    public function build($runData)
    {

        // check first for standard cookie name
        $user = $runData->getUser();
        $anon = false;

        if (! $user) {
            // check the ie cookie then
            GlobalProperties::$SESSION_COOKIE_NAME = GlobalProperties::$SESSION_COOKIE_NAME_IE;
            $runData->handleSessionStart();
            $user = $runData->getUser();
            $anon = ($_COOKIE[GlobalProperties::$SESSION_COOKIE_NAME_IE] == "ANONYMOUS");
        }

        if ($user) {
            $site_id = (int) $runData->getParameterList()->getParameterValue("site_id");
            $site = SitePeer::instance()->selectByPrimaryKey($site_id);

            if ($site && $site->getCustomDomain()) {
                $skey = $runData->generateSessionDomainHash($site->getCustomDomain());
                $proto = ($_SERVER["HTTPS"]) ? "https" : "http";
                $domain = $site->getCustomDomain();
                $runData->contextAdd("redir", "$proto://$domain" . CustomDomainLoginFlowController::$controllerUrl . "?" . http_build_query(array("user_id" => $user->getUserId(), "skey" => $skey)));
            }
        } elseif (! $anon) {
            // no session found -- try to redirect to set ie cookie
            $proto = ($_SERVER["HTTPS"]) ? "https" : "http";
            $runData->contextAdd("redirIE", $proto . '://' . GlobalProperties::$URL_HOST . CustomDomainLoginFlowController::$controllerUrl . '?' . http_build_query(array("url" => $url, "setiecookie" => true)));
        }
    }
}
