<h1>{t}Account settings{/t}</h1>


<div>
	<h3><a href="javascript:;" onclick="OZONE.ajax.requestModule('account/settings/ASEmailModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}My email address{/t}</a></h3>
	<p>
		{t}Your primary email address is now{/t}: {$user->getEmail()|escape}. 
	</p>
</div>

<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/settings/ASPasswordModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Change password{/t}</a></h3>
	<p>
		{t}Simply change your access password if you need or want.{/t}
	</p>
</div>



<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/settings/ASNotificationsModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Notifications - online &amp; private RSS &amp; email{/t}</a></h3>
	<p>
		{t}Configure the way Wikidot informs you about events related to your presence here.{/t}
	</p>
</div>

{*
<div>
	<h3><a href="javascript:;"  onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-wiki-newsletters')">{t}Wiki Newsletters{/t}</a></h3>
	<p>
		{t}Tell us if you want to receive email newsletters from the Wikis you are a member of.{/t}
	</p>
</div>
*}
<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/settings/ASMessagesModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Receiving private messages{/t}</a></h3>
	<p>
		{t}Is everybody allowed to send you a private message? Change this setting if you wish.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/settings/ASInvitationsModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Receiving invitations{/t}</a></h3>
	<p>
		{t}Do you want to receive invitations to participate in other sites?{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/settings/ASBlockedModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Blocked users{/t}</a></h3>
	<p>
		{t}Configure the list of users you do not want to hear or receive anything from.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/settings/ASLanguageModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Preferred language{/t}</a></h3>
	<p>
		{t}Choose the language you would prefer to use.{/t}
	</p>
</div>