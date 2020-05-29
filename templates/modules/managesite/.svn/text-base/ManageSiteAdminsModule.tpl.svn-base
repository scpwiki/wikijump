<h1>{t}Site Administrators{/t}</h1>

{foreach from=$admins item=admin}
	{assign var=user value=$admin->getUser()}
	<div>
		<div style="position: absolute; margin-left: 30em">
			<a href="javascript:;" onclick="removeAdmin({$user->getUserId()}, '{$user->getNickName()}')">{t}remove from administrators{/t}</a>
		</div>
		{printuser user=$user image="yes"} 
	</div>
{/foreach}

<div id="remove-admin-dialog" style="display: none">
	{t}Are you sure you want to remove <strong>%%USER_NAME%%</strong> from site administrators?{/t}
</div>	