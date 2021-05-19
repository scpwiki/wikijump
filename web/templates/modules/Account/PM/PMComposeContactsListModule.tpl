<div class="owindow" style="width: 60%">
	<div class="title">
		{t}Contacts list{/t}
	</div>
	<div class="content">
		<h1>{t}Choose the recipient from your contacts{/t}</h1>

		<div id="pm-contacts-list" style="width: 60%;">
			{if isset($contacts)}
				<ul style="list-style: none">
					{foreach from=$contacts item=contact}
						{assign var=user value=$contact->getTargetUser()}
						<li>
							<span class="printuser"><img class="small" src="{$contact->getTemp("avatarUri")}"/>
							<a href="javascript:;" onclick="OZONE.dialog.cleanAll();Wikijump.modules.PMComposeModule.utils.selectRecipient({$user->id}, '{$user->username|escape}');" >{$user->username|escape}</a>
						</li>
					{/foreach}
				</ul>
			{else}
				<p>{t}You have no users in your contact list.{/t}</p>
			{/if}
		</div>
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}cancel{/t}</a>
	</div>
</div>
