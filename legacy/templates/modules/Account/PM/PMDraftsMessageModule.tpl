{loadmacro set="PM"}

{if $newerMessage != null|| $olderMessage!= null}
	<div style="text-align: right" id="inbox-message-nav">
		{if $newerMessage}<a href="javascript:;" onclick="Wikijump.modules.AccountMessagesModule.listeners.viewDraftsMessage({$newerMessage->id})">{t}newer{/t}</a> {/if}
		{if $newerMessage != null && $olderMessage!= null}|{/if}
		{if $olderMessage}<a href="javascript:;" onclick="Wikijump.modules.AccountMessagesModule.listeners.viewDraftsMessage({$olderMessage->id})">{t}older{/t}</a>{/if} {t}message (by date){/t}
	</div>
{/if}

{macro name="pm" message=$message to=true}

<div id="inbox-message-options">
	 <a href="javascript:;" onclick="Wikijump.modules.PMDraftsModule.listeners.editDraftMessage(event, {$message->id})">{t}edit as new{/t}</a>
	| <a href="javascript:;" onclick="Wikijump.modules.PMDraftsModule.listeners.removeDraftsMessage(event, {$message->id})">{t}delete message{/t}</a>
</div>

