<?php



require_once (SMARTY_DIR . '/Smarty_Compiler.class.php');

/**
 * Modified compiler for the Smarty.
 *
 */
class OzoneSmartyCompiler extends Smarty_Compiler {

	/**
	 * Allow static method calls.
	 */
	function _parse_attrs($tag_args) {
		$attrs = parent :: _parse_attrs($tag_args);

		foreach ($attrs as $key => $value) {
			// perhaps this was intended as a static callback?
            if (preg_match('/
                ^["\']                         # Quote at start
                ([a-zA-Z_]\w*::[a-zA-Z_]\w*)   # Two words split by ::
                \((.*)?\)                      # Anything else, in parens
                ["\']$                         # Closing quote
                /x', $value, $matches)) {
				$arguments = '()';
				if (isset ($matches[2])) {
					// strip '".' and '."' from beginning and end
					$arguments = substr($matches[2], 2, -2);

					// remove '.",".' from between parameters
					$arguments = explode('.",".', $arguments);

					// combine arguments into string
					$arguments = '(' . implode(',', $arguments) . ')';
				}

				$attrs[$key] = $matches[1] . $arguments;
			}
		}
		return $attrs;
	}

}
