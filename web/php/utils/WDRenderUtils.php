<?php
class WDRenderUtils
{

    public static function renderUser($user, $params = array())
    {

        if ($user == null || $user == '') {
            return null;
        }
        if (is_string($user)) {
            $linkInner = 'href="javascript:;"  onclick="Wikijump.page.listeners.anonymousUserInfo(\''.$user.'\'); return false;" ';
            //  ok, this is just "anonymous info". print it!
            $out = '<span class="printuser anonymous">';
            if ($params['image'] != null) {
                $image = $params['image'];
                // handle sizes...
                $out .=     '<a '.$linkInner.' ><img class="small" src="/common--images/avatars/default/a16.png" alt=""/></a>';
            }
            $out .= '<a '.$linkInner.'>'._('Anonymous');
            list($ip, $proxy) = explode("|", $user);

            if (!$params['noip']) {
                $out .= ' <span class="ip">('.htmlspecialchars($ip).')</span>';
            }

            $out .= '</a></span>';
            return $out;
        }

        $userId = $user->getUserId();

        if ($userId<0) {
            // always mean some kind of system bot. just print bot name.
            $out = '<span class="printuser">'.htmlspecialchars($user->getNickName()).'</span>';
            return $out;
        }

        $class = "printuser";
        if ($params['image'] && $params['image'] !== 'small') {
            // TODO Remove avatar hover (WJ-224)
            $class .= " avatarhover";
        }

        $out = '<span class="'.$class.'">';
        $linkInner = 'href="'.GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/user:info/'.$user->getUnixName().'" onclick="Wikijump.page.listeners.userInfo('.$user->getUserId().'); return false;" ';
        if ($params['image'] != null) {
            $image = $params['image'];
            // handle sizes...
            $out .=     '<a '.$linkInner.' ><img class="small" src="/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a16.png" alt="'.htmlspecialchars($user->getNickName()).'"';
            /* karma: */
            $out .= ' style="background-image:url('.GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/userkarma.php?u=' . $userId . ')"';
            /* end of karma */
            $out .= '/></a>';
        }
        if (!$params['noNameLink']) {
            $out .= '<a '.$linkInner.'>'.htmlspecialchars($user->getNickName()).'</a></span>';
        } else {
            htmlspecialchars($user->getNickName());
        }
        return $out;
    }
}
