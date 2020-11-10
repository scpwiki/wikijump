<?php



/**
 * Prefirter required for defmacro
 */
function smarty_prefilter_defmacrohelp($source, &$smarty) {
    $out = preg_replace('({defmacro[^}]*})', ' $0 {literal} ', $source);
    $out = preg_replace('({/defmacro})', '{/literal} {/defmacro} ', $out);
    return $out;
}
