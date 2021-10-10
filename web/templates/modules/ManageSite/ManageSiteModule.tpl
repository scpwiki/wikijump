{if $start}
<script type="text/javascript">
	smStartPage = "{$start}";
</script>
{/if}

{strip}
<div id="site-manager">
	<div id="site-manager-menu">
		<div class="head">
			{t}manage site...{/t}
		</div>
		<ul id="sm-menu">
			<li><a href="javascript:;" id="sm-welcome">{t}Welcome to site manager{/t}!</a></li>
			<li><a href="javascript:;" id="sm-general">{t}General settings{/t}</a></li>
			<li><a href="javascript:;" id="sm-domain">{t}Custom domain{/t}</a></li>
			{if $allowHttp}
			{if $useSsl}
				<li><a href="javascript:;" id="sm-ssl">{t}Secure access (SSL/TLS){/t}</a></li>
			{/if}
			{/if}
			<li><a href="javascript:;">{t}Appearance{/t}</a>
				<ul>
					<li><a href="javascript:;" id="sm-appearance">{t}Themes{/t}</a></li>
					<li><a href="javascript:;" id="sm-customthemes">{t}Custom themes{/t}</a></li>
					<li><a href="javascript:;" id="sm-navigation">{t}Navigation elements{/t}</a></li>
				</ul>
			</li>
			<li><a href="javascript:;" id="sm-license">{t}License{/t}</a></li>
			<li><a href="javascript:;" id="sm-templates">{t}Page templates{/t}</a></li>
			<li><a href="javascript:;" id="sm-private">{t}Public or private{/t}</a></li>
			<li><a href="javascript:;" id="sm-permissions">{t}Permissions{/t}</a></li>
			<li><a href="javascript:;" id="sm-files">{t}Files{/t}</a></li>
			<li><a href="javascript:;">{t escape=no}Forum &amp; discussion{/t}</a>
				<ul>
					<li><a href="javascript:;" id="sm-forum-settings">{t}Settings{/t}</a></li>
					<li><a href="javascript:;" id="sm-forum-layout">{t}Structure{/t}</a></li>
					<li><a href="javascript:;" id="sm-forum-perpage">{t}Per page discussion{/t}</a></li>
					<li><a href="javascript:;" id="sm-forum-perm">{t}Permissions{/t}</a></li>
					<li><a href="javascript:;" id="sm-forum-recent">{t}Recent posts &amp; comments{/t}</a></li>
				</ul>
			</li>
			<li><a href="javascript:;">{t}Members{/t}</a>
				<ul>
					<li><a href="javascript:;" id="sm-members">{t}Policy{/t}</a></li>
					<li><a href="javascript:;" id="sm-ma">{t}Applications{/t}</a></li>
					<li><a href="javascript:;" id="sm-members-list">{t}List Members{/t}</a></li>
					<li><a href="javascript:;" id="sm-moderators">{t}List Moderators{/t}</a></li>
					<li><a href="javascript:;" id="sm-admins">{t}List Admins{/t}</a></li>
					<li><a href="javascript:;" id="sm-members-invite">{t}Invite Members{/t}</a></li>
					<li><a href="javascript:;" id="sm-email-invitations">{t}Send Email Invitations{/t}</a></li>
					<li><a href="javascript:;" id="sm-invitations-history">{t}History of Invitations{/t}</a></li>
					<li><a href="javascript:;" id="sm-users-email-invitations">{t}Let the Users invite{/t}</a></li>
				</ul>
			</li>
			{if $enableLists}
			<li><a href="javascript:;" id="sm-email-lists">{t}Email lists{/t}</a></li>
			{/if}
			<li><a href="javascript:;">{t}Blocks{/t}</a>
				<ul>
					<li><a href="javascript:;" id="sm-ip-blocks">{t}IP blocks{/t}</a></li>
					<li><a href="javascript:;" id="sm-user-blocks">{t}User blocks{/t}</a></li>
				</ul>
			</li>
			<li><a href="javascript:;" id="sm-recent-changes">{t}Recent changes{/t}</a></li>
			<li><a href="javascript:;" id="sm-pagerate">{t}Page ratings{/t}</a></li>
			<li><a href="javascript:;">{t}Abuse reports{/t}</a>
				<ul>
					<li><a href="javascript:;" id="sm-abuse-page">{t}Pages{/t}</a></li>
					<li><a href="javascript:;" id="sm-abuse-user">{t}Users{/t}</a></li>
					<li><a href="javascript:;" id="sm-abuse-anonymous">{t}Anonymous Users{/t}</a></li>
				</ul>
			</li>
			<li><a href="javascript:;" id="sm-notifications">{t}Notifications{/t}</a></li>
		</ul>
		<hr/>

		<ul>
			<li><a href="javascript:;">{t}Extreme actions{/t}</a>
				<ul>
					<li><a href="javascript:;" id="sm-renamesite">{t}Change base URL{/t}</a></li>
					<li><a href="javascript:;" id="sm-deletesite">{t}Delete this Site{/t}</a></li>
					<li><a href="javascript:;" id="sm-clonesite">{t}Clone{/t}</a></li>
				</ul>
			</li>
		</ul>
	</div>
	<div id="sm-action-area">
		{t}Loading...{/t}
	</div>
</div>
{/strip}
