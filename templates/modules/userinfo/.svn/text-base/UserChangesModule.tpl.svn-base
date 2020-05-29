<h1>{t}Recent contributions{/t}</h1>

<div class="user-changes-box" >
	<form onsubmit="return false;" action="dummy.html" method="get">
		<input type="hidden" id="changes-user-id" value="{$userId}"/>
		<table class="form">
			<tr>
				<td>
					{t}Revision types{/t}:
				</td>
				<td>
					<input class="checkbox" type="checkbox" id="rev-type-all" checked="checked"/>&nbsp;{t}ALL{/t}<br/>
					<input class="checkbox" type="checkbox" id="rev-type-new"/>&nbsp;{t}new pages{/t}<br/>
					<input class="checkbox" type="checkbox" id="rev-type-source"/>&nbsp;{t}source{/t}<br/>
					<input class="checkbox" type="checkbox" id="rev-type-title"/>&nbsp;{t}title{/t}<br/>
					<input class="checkbox" type="checkbox" id="rev-type-move"/>&nbsp;{t}move/rename{/t}<br/>
					<input class="checkbox" type="checkbox" id="rev-type-meta"/>&nbsp;{t}meta data{/t}<br/>
					<input class="checkbox" type="checkbox" id="rev-type-files"/>&nbsp;{t}attachments (files){/t}
				</td>
			</tr>
			<tr>
				<td>
					{t}Revisions per page{/t}:
				</td>
				<td>
					<select id="rev-perpage">
						<option value="10">10</option>
						<option value="20" selected="selected">20</option>
						<option value="50">50</option>
						<option value="100">100</option>
						<option value="200">200</option>
					</select>
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input class="button" type="button" value="{t}update list{/t}" onclick="WIKIDOT.modules.UserChangesModule.listeners.updateList(null)"/>
		</div>
	</form>
	<div class="changes-list" id="site-changes-list">
		{module name="userinfo/UserChangesListModule" userId=$userId}
	</div>
</div>