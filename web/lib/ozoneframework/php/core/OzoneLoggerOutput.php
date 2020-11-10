<?php





/**
 * Output stream interface for the logger.
 *
 */
interface OzoneLoggerOutput {
	/**
	 * Handles event sent by OzoneLogger.
	 */
	public function handleEvent($event);

}
