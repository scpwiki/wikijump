{if $error}
	{if $error == 'user_not_exist'}
		<div class="errorbox"><div>{t 1=$userUnixName}User %1 does not exist.{/t}</div></div>
	{/if}
{else}
{strip}
<div id="user-info-side">
	<div class="head">{$user->getNickName()|escape}</div>
	<hr/>
	<ul>
		<li><a id="ui-profile-b" class="active" href="javascript:;">{t}Profile{/t}</a></li>
		<li><a id="ui-member-b" href="javascript:;">{t}Member of{/t}</a></li>
		<li><a id="ui-moderator-b" href="javascript:;">{t}Moderator of{/t}</a></li>
		<li><a id="ui-admin-b" href="javascript:;">{t}Admin of{/t}</a></li>
		<li><a id="ui-contrib-b" href="javascript:;">{t}Recent contributions{/t}</a></li>
		<li><a id="ui-posts-b" href="javascript:;">{t}Recent posts/comments{/t}</a></li>
	</ul>
	<br/><br/>
	<ul>
		<li><a href="http://{$URL_HOST}/account:you/start/messages/composeto/{$user->getUserId()}">{t}Write PM{/t}</a></li>
		<li><a href="javascript:;" onclick="WIKIDOT.modules.UserInfoModule.listeners.addContact(event,{$userId})">{t}Add to contacts{/t}</a></li>
	</ul>
	
	<br/><br/><br/>
	<ul>
		<li><a href="javascript:;" style="color: #AAD" onclick="WIKIDOT.modules.UserInfoModule.listeners.flagUser(event,{$userId})">{t}flag user as abusive{/t}</a></li>
	</ul>
	
	
	
</div>
{/strip}
<div id="user-info-area">

{module name="userinfo/UserInfoProfileModule" user_id="$userId"}
{* {$module->render("userinfo/UserInfoProfileModule", 'user_id="$userUnixName"')} *}

</div>

{/if}

<script type="text/javascript">
	{literal}USERINFO = {};{/literal}
	USERINFO.userId = {$userId};
	{if $referer}
		USERINFO.referer = "{$referer}";
	{/if}
</script>