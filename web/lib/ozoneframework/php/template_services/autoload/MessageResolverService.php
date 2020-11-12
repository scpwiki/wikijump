<?php



/**
 * Message resolver service.
 *
 */
class MessageResolverService extends TemplateService{

	protected $serviceName = "messageService";

	public function message($key){
		return MessageResolver::instance()->message($key);
	}

}
