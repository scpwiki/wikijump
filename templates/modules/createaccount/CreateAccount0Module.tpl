{loadmacro set="Forms"}


<div class="owindow" style="width: 60em">

	<div class="title">
		{t}Create account: account information{/t}
	</div>
	<div class="content">
		<h1>{$SERVICE_NAME} account data</h1>
		
		<div class="error-block" id="ca-reg0-errors" style="display: none"></div>
		
		<form name="caform" action="dummy.html" method="get" onkeypress="return OZONE.utils.disableEnterKey(event)" id="createaccount-form0" onsubmit="return false">
			<table class="form"> 
				<tr>
					<td>
						{t}Your screen name{/t}:
					</td>
					<td>
						<input class="text" type="text" maxlength="50" size="25" name="name"/>
						<div class="sub">
							{t 1=$SERVICE_NAME}That is your new name at %1<br/>
							your identity.{/t}
						</div>
					</td>
				</tr>
				<tr>
					<td>
						{t}Your email address{/t}:
					</td>
					<td>
						<input class="text" type="text"  maxlength="50" size="25" name="email"/>
					</td>
				</tr>
				<tr>
					<td>
						{t}Preferred language{/t}:
					</td>
					<td>
						<input type="radio" name="language" value="en" id="new-site-lang-en"/> <label for="new-site-lang-en">{t}English{/t}</label>
						&nbsp;&nbsp;&nbsp;&nbsp;
						<input type="radio" name="language" value="pl" id="new-site-lang-pl"/> <label for="new-site-lang-pl">{t}Polish{/t}</label>
					
					</td>
				</tr>
				
		{*	</table>
			<h2>Account access password </h2>
			<table  class="form">	*}
				<tr>
					<td>
						{t}Password{/t}:
					</td>
					<td>
						 <input class="text" type="password" name="password" maxlength="30" size="15"/>
						 <div class="sub">
						 	{t}Between 6 and 20 characters{/t}.
						 </div>
					</td>
				</tr>
				<tr>
					<td>
						{t}Password (repeat){/t}:
					</td>
					<td>
						 <input class="text" type="password" name="password2" maxlength="30" size="15"/>
					</td>
				</tr>
				<tr>
					<td>
						{t}Are you a human?{/t}
					</td>
					<td>
						<img src="/default--flow/misc__Captcha/rand/{$rand}" alt="turing code"/>
						<br/>
						<input class="text" type="text" name="captcha" size="10" />
						<div class="sub">
							{t}Please write the code you can see above. Upper/lowercase does not matter.{/t}
							<br/>{t}This is to prevent automated bots.{/t}
						</div>
					</td>
				</tr>
				<tr>
					<td>
						{t}Please confirm:{/t}
					</td>
					<td>
						<input type="checkbox" name="tos" class="checkbox">
						
						<!-- TODO: De-Wikidot.com-ize - change -->
						{t 1=$URL_HOST}I have read and agree to the <a href="http://%1/legal:terms-of-service"
						target="_blank">Terms of Service</a>.{/t}
						
					</td>
					
				</tr>
			</table>
		</form>
		<div class="note">
			<ul>
				<li>
					{t}Your account data will be sent using a secure method.{/t}
				</li>
				<li>
					{t}Please give the email address you have access to. Email + password will be used later to log in.{/t}
				</li>
			</ul>
		</div>
		
		
	</div>
	<div class="button-bar">
		<a href="javascript:;"  onclick="WIKIDOT.modules.CreateAccountModule.listeners.cancel(event)">{t}cancel{/t}</a>
		<a href="javascript:;" onclick="WIKIDOT.modules.CreateAccount0Module.listeners.nextClick(event)">{t}next{/t}</a>
	</div>
</div>

