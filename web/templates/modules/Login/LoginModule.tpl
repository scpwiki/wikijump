
<div class="error-block" id="loginerror" style="display: none"></div>
<form id="login-form" action="common--html/dummy.html" method="post"
	onsubmit="Wikijump.modules.LoginModule.listeners.loginClick(event)">

	{if isset($user)}
		{t}Hello{/t}, <span style="font-size:130%; font-weight: bold">{$user->username|escape}</span>
        <input type="hidden" name="name" value="{$user->username|escape}" id="login-form-name"/>
		<br/>
		<br/>
	{else}
		{t}Email or nick name{/t}
		<br/>
		<input class="text" name="name" type="text" size="25" id="login-form-name"/>
		<br/><br/>
	{/if}
	{t}Password{/t}
	<br/>
	<input class="text" name="password"  type="password" size="25" id="login-form-password"/>
	<br/><br/>
	{if isset($user)}
		Not {$user->username|escape}?<br/>
		<a href="javascript:;" style="font-size: 85%" onclick="Wikijump.modules.LoginModule.listeners.switchUser(event)">{t}Log in as a different user{/t}</a>.
		<br/><br/>
	{/if}

	<input class="checkbox" name="keepLogged" type="checkbox"
		id="login-form-keeplogged" checked="checked"/>
	<label for="login-form-keeplogged">{t}Keep me logged in unless I log out{/t}</label> <span id="keep-logged-info">[?]</span>
	<br/>
	&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;(Uncheck if on a shared computer)

	<br/>
	<input class="checkbox" name="bindIP" type="checkbox" checked="checked"
		id="login-form-bindip"/>
	<label for="login-form-bindip">{t}Bind session to my IP{/t}</label> <span id="bind-ip-info">[?]</span>


	<hr/>
	<p>
		<a href="javascript:;" onclick="Wikijump.page.listeners.passwordRecoveryClick(event)">{t}Forgot your password?{/t}</a>
	</p>
	<hr/>
	<p>
		<a href="/auth:newaccount">{t}No account yet? Get one!{/t}</a>
	</p>
	<div class="buttons" >
		<input type="button" onclick="Wikijump.modules.LoginModule.listeners.cancel(event)" value="{t}cancel{/t}"/>
		<input type="submit" value="{t}login{/t}" style="font-weight: bold"/>
	</div>
	<div id="keep-logged-info-hovertip">
			{t}Select this option and you will not be automatically logged-out after 30 minutes
			of inactivity.{/t}
		</div>
		<div id="bind-ip-info-hovertip">
			{t}In order to increase security it is advised to bind the session to the IP address of the computer you use.
			Problems? When your computer changes the IP number (via DHCP) you might loose your session
			(and log out) occasionally. Option recommended.{/t}
		</div>
</form>

