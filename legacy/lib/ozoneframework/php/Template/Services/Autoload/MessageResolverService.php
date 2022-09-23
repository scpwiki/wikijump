<?php

namespace Ozone\Framework\Template\Services\Autoload;



use Ozone\Framework\MessageResolver;
use Ozone\Framework\TemplateService;

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
