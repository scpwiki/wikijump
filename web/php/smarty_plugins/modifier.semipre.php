<?php
function smarty_modifier_semipre($string)
{
    $out = $string;
    $out = preg_replace("/^\t/m", '    ', $out);
    $out = preg_replace_callback('/^ +/m', smarty_modifier_semipre_callback1, $out);
    $out = preg_replace_callback('/ {2,}/m', smarty_modifier_semipre_callback1, $out);
    $out = nl2br($out);
    return $out;
}

function smarty_modifier_semipre_callback1($matches)
{
    $string = $matches[0];
    return str_repeat('&nbsp;', strlen($string));
}
