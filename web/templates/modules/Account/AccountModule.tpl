{if isset($start)}
<script type="text/javascript">
	accountStartPage = "{$start}";
</script>
{/if}
{if isset($composeTo)}
<script type="text/javascript">
	composeTo = "{$composeTo}";
</script>
{/if}
{if isset($inboxMessage)}
<script type="text/javascript">
	inboxMessage = "{$inboxMessage}";
</script>
{/if}
{if isset($rsaKey)}
	<script type="text/javascript">
		Wikijump.vars.rsakey = "{$rsaKey}";
	</script>
{/if}
{strip}
<div id="account-box">
<div id="account-side">
	<div class="head">
		{t}My account{/t}
	</div>
	<ul>
		<li><a href="javascript:;" id="am-welcome">{t}Welcome{/t}!</a></li>

		<li><a href="javascript:;" id="am-messages" >{t}Private messages{/t}</a></li>
		<li><a href="javascript:;" id="am-notifications">{t}Notifications{/t}</a></li>
	</ul>
	<hr/>
	<ul>
		<li><a href="javascript:;" id="am-contacts" >{t}My contacts{/t}</a></li>
	</ul>
	<hr/>
	<ul>
		<li><a href="javascript:;">{t}Watched...{/t}</a>
			<ul>
				<li><a href="javascript:;" id="am-watched-changes">{t}Pages{/t}</a></li>
				<li><a href="javascript:;" id="am-watched-forum">{t}Discussions{/t}</a></li>
				<li><a href="javascript:;" id="am-watched-feed">{t}Via RSS feed{/t}</a></li>
			</ul>
		</li>
	</ul>
	<hr/>
	<ul>




		<li><a href="javascript:;">{t escape=no}Sites &amp; membership{/t}</a>
			<ul>
				<li><a href="javascript:;" id="am-memberof">{t}Member of{/t}</a></li>
				<li><a href="javascript:;" id="am-moderatorof">{t}Moderator of{/t}</a></li>
				<li><a href="javascript:;" id="am-adminof">{t}Admin of{/t}</a></li>
				<li><a href="javascript:;" id="am-deletedsites">{t}Deleted Sites{/t}</a></li>
				<li><a href="javascript:;" id="am-invitations">{t}Invitations{/t}</a></li>
				<li><a href="javascript:;" id="am-applications">{t}Applied to{/t}</a></li>
				{*<li><a href="javascript:;" id="am-wiki-newsletters">{t}Newsletters{/t}</a></li>*}
			</ul>
		</li>

	</ul>
	<hr/>
	<ul>
		<li><a href="javascript:;">{t}Recent activity{/t}</a>
			<ul>
				<li><a href="javascript:;" id="am-recentcontrib">{t}Recent contributions{/t}</a></li>
				<li><a href="javascript:;" id="am-recentposts">{t}Recent posts/comments{/t}</a></li>
			</ul>
		</li>
		{*<li><a href="javascript:;"  id="am-stats">{t}Overall statistics{/t}</a></li>*}
	</ul>
	<hr/>
	<ul>
		<li><a href="javascript:;" id="am-profile">{t}My profile{/t}</a></li>
		<li><a href="javascript:;" id="am-settings">{t}Account settings{/t}</a></li>
	</ul>
</div>
{/strip}
<div id="account-area">
	<div class="wait-block">{t}Loading...{/t}</div>
</div>
</div>
<script type="text/javascript">
	{literal}USERINFO = {};{/literal}
	USERINFO.userId = {$user->id};
</script>
