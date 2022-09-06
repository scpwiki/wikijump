<?php

namespace Wikidot\Utils;


use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;

class CustomDomainLoginFlowController extends WikidotController
{

    public static $controllerUrl = "/domainauth.php";

    protected function redirectConfirm($url)
    {
        $this->redirect(self::$controllerUrl, array("confirm" => "cookie", "url" => $url));
    }

    protected function cookieError($url)
    {
        $url = htmlspecialchars($url);
        $this->setContentTypeHeader("text/html");
        echo "<p>Can't proceed, you should accept cookies for this domain.</p>";
        echo "<p>Then you can go back to $url</p>";
    }

    public function process()
    {

        Ozone ::init();

        $runData = new RunData();
        $runData->init();
        Ozone::setRunData($runData);

        $url = $_GET["url"];
        $confirm = isset($_GET["confirm"]);
        $setie = isset($_GET["setiecookie"]);
        $siteHost = $_SERVER['HTTP_HOST'];

        $site = $this->siteFromHost($siteHost, true, true);

        if ($setie) {
            if ($siteHost != GlobalProperties::$URL_DOMAIN) {
                $this->siteNotExists();
            }

            $runData->handleSessionStart();
            if ($runData->getUser()) {
                setsecurecookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), 0, '/', GlobalProperties::$SESSION_COOKIE_DOMAIN);
            } else {
                setsecurecookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, "ANONYMOUS", 0, '/', GlobalProperties::$SESSION_COOKIE_DOMAIN);
            }
            $this->redirect($url);
        } else {
            if (! $site) {
                $this->siteNotExists();
                return;
            }

            if (! $confirm) {
                $user_id = $_GET["user_id"];
                $skey =  $_GET["skey"];

                $session = $runData->getSessionFromDomainHash($skey, $_SERVER['HTTP_HOST'], $user_id);

                if ($session) {
                    if(GlobalProperties::$SESSION_COOKIE_SECURE == true) {
                        setsecurecookie(GlobalProperties::$SESSION_COOKIE_NAME_SSL, "_domain_cookie_${user_id}_${skey}", 0, '/', GlobalProperties::$SESSION_COOKIE_DOMAIN);
                    }
                    else {
                        setsecurecookie(GlobalProperties::$SESSION_COOKIE_NAME, "_domain_cookie_${user_id}_${skey}", 0, '/', GlobalProperties::$SESSION_COOKIE_DOMAIN);
                    }
                    $this->redirectConfirm($url);
                } else {
                    $this->redirect($url);
                }
            } else {
                // checking if cookie exists

                $runData->handleSessionStart();
                $this->redirect($url);
            }
        }
    }
}
