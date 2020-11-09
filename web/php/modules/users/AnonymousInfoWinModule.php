<?php
class AnonymousInfoWinModule extends SmartyModule
{

    public function build($runData)
    {
        $userString = $runData->getParameterList()->getParameterValue("userString");

        // check if matches.
        # Possibly validates some internal user ID
        if (preg_match("/^((?:[0-9]{1,3}\.){3}[0-9]{0,3})(?:\|((?:[0-9]{1,3}\.){3}[0-9]{0,3}))?$/", $userString) == 0) {
            throw new ProcessException("Bad data");
        }

        list($ip, $proxy) = explode("|", $userString);

        $runData->contextAdd("ip", $ip);
        $runData->contextAdd("proxy", $proxy);

        // check if IP comes from a private range
        // 10.*.*.*, 172.16.*.*,  192.168.*.*, 127.*.*.*, 169.254.*.*

        if (preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0) {
            $runData->contextAdd("privateIp", true);
        }

        $runData->contextAdd("userString", $userString);
    }
}
