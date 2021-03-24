<?php

use Wikidot\Utils\WDRenderUtils;

function smarty_function_printuser($params, &$smarty)
{
    $user = $params['user'];
    return WDRenderUtils::renderUser($user, $params);
}
