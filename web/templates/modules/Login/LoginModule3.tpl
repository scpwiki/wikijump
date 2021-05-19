<div id="login-window" class="owindow"
	style="width: 100%; position: static; margin:0; padding:0; border: none;">
	<div class="content" style="width: auto; position: static; margin:0; padding:0; border: none;">
		<h2 id="login-head">{t}Login{/t}</h2>
		<div class="error-block" id="loginerror" style="display: none"></div>
		<form id="login-form" action="/common--misc/blank.html" method="post"
			{*onsubmit="Wikijump.modules.LoginModule3.listeners.loginClick(event);"*}>
			<div style="text-align: center">
				{if isset($user)}
					{t}Hello{/t}, <span style="font-size:130%; font-weight: bold">{$user->username|escape}</span>
					<br/>
					<br/>
				{else}
					{t}Email{/t}
					<br/>
					<input class="text" name="loginName" type="text" size="25" id="login-form-name"/>
					<br/><br/>
				{/if}
				{t}Password{/t}
				<br/>
				<input class="text" name="password"  type="password" size="25" id="login-form-password"/>
				<br/><br/>
				{if isset($user)}
					{*(<a href="javascript:;" style="font-size: 85%" onclick="window.location.href = window.location.href+'/clearwelcome/true'">{t}log in as a different User{/t}</a>)*}
					(<a href="javascript:;" style="font-size: 85%" onclick="this.href = window.location.href+'/clearwelcome/true'">{t}log in as a different User{/t}</a>)
					<br/><br/>
				{/if}
				<label for="login-form-keeplogged"><strong>{t}Do not timeout my session{/t}</strong></label>
				<input class="checkbox" name="keepLogged" type="checkbox"
					id="login-form-keeplogged"/> <span id="keep-logged-info">[?]</span>
				<br/>
				<label for="login-form-bindip">{t}Bind session to my IP{/t}</label>
				<input class="checkbox" name="bindIP" type="checkbox" checked="checked"
					id="login-form-bindip"/> <span id="bind-ip-info">[?]</span>
			</div>
			<div class="buttons" id="login-buttons">
				<input type="button" onclick="top.location.href='{$backUrl}'" value="{t}cancel{/t}"/>
				<input type="submit" value="{t}login{/t}" style="font-weight: bold"/>
			</div>
			<p class="wait-progress" id="login-progress" style="display:none; padding-top:40px; text-align: center;">
				<input type="button" class="button" onclick="top.location.href='{$backUrl}'" value="{t}cancel{/t}"/>
			</p>
		</form>


		<div id="keep-logged-info-hovertip">
			{t}Select this option and you will not be automatically logged-out after 30 minutes
			of inactivity.{/t}
		</div>
		<div id="bind-ip-info-hovertip">
			{t}In order to increase security it is advised to bind the session to the IP address of the computer you use.
			Problems? When your computer changes the IP number (via DHCP) you might loose your session
			(and log out) occasionally. Option recommended.{/t}
		</div>
	</div>
</div>
