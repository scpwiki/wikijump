{loadmacro set="PM"}

{if $newerMessage != null|| $olderMessage!= null}
	<div style="text-align: right" id="inbox-message-nav">
		{if isset($newerMessage)}<a href="javascript:;" onclick="Wikijump.modules.AccountMessagesModule.listeners.viewInboxMessage({$newerMessage->getMessageId()})">{t}newer{/t}</a> {/if}
		{if isset($newerMessage) && isset($olderMessage)}|{/if}
		{if isset($olderMessage)}<a href="javascript:;" onclick="Wikijump.modules.AccountMessagesModule.listeners.viewInboxMessage({$olderMessage->getMessageId()})">{t}older{/t}</a>{/if} {t}message (by date){/t}
	</div>
{/if}

{macro name="pm" message=$message from=true}

<div id="inbox-message-options">
	<a href="javascript:;" onclick="Wikijump.modules.AccountMessagesModule.listeners.replyInboxMessage(event, {$message->getMessageId()})">{t}reply{/t}</a>
	| <a href="javascript:;" onclick="Wikijump.modules.PMInboxModule.listeners.removeInboxMessage(event, {$message->getMessageId()})">{t}delete message{/t}</a>
</div>

<div id="pm-reply-area"></div>
