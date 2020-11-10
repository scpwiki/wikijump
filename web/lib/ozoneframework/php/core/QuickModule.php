<?php





/**
 * Abstract class for a quick module.
 */
abstract class QuickModule {
	/**
	 * Process the input data. $data is the parsed (json) post body of the request.
	 */
	public abstract function process($data);

}
