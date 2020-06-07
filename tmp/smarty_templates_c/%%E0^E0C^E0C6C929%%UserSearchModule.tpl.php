<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:19
         compiled from /var/www/wikidot/templates/modules/search/UserSearchModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/modules/search/UserSearchModule.tpl', 8, false),array('block', 't', '/var/www/wikidot/templates/modules/search/UserSearchModule.tpl', 9, false),array('function', 'printuser', '/var/www/wikidot/templates/modules/search/UserSearchModule.tpl', 28, false),array('function', 'pager', '/var/www/wikidot/templates/modules/search/UserSearchModule.tpl', 50, false),)), $this); ?>
<div class="search-box">


	<div class="query-area">
		<form action="javascript:;" id="search-form-user">
			<div>

				<input class="text" type="text" size="30" name="query" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['query'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
"/>
				<input class="button" type="submit" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>search<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>"/>
			</div>
		</form>
	</div>

	<?php if ($this->_tpl_vars['errorMessage']): ?>
		<p><?php echo $this->_tpl_vars['errorMessage']; ?>
</p>
	<?php endif; ?>

	<?php if ($this->_tpl_vars['mode'] == 'email'): ?>

		<?php if ($this->_tpl_vars['user']): ?>

			<?php $this->assign('profile', $this->_tpl_vars['user']->getProfile()); ?>
			<div class="search-user-results">
				<div class="item">
					<div class="screen-name">
						<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['user'],'image' => 'true'), $this);?>

						<?php if ($this->_tpl_vars['profile']->getRealName() != ''): ?> (real name: <?php echo ((is_array($_tmp=$this->_tpl_vars['profile']->getRealName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
)<?php endif; ?>
					</div>
					<?php if ($this->_tpl_vars['profile']->getAbout() != ''): ?>
						<div class="about">
							<?php echo ((is_array($_tmp=$this->_tpl_vars['profile']->getAbout())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>

						</div>
					<?php endif; ?>
				</div>
			</div>

		<?php else: ?>
			<p>
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Sorry, no user with such an email address.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
			</p>
		<?php endif; ?>

	<?php else: ?>



		<?php ob_start(); ?>/search:users/q/<?php echo $this->_tpl_vars['queryEncoded']; ?>
/p/%d<?php $this->_smarty_vars['capture']['destUrl'] = ob_get_contents(); ob_end_clean(); ?>
		<?php echo smarty_function_pager(array('url' => $this->_smarty_vars['capture']['destUrl'],'total' => $this->_tpl_vars['pagerData']['total_pages'],'known' => $this->_tpl_vars['pagerData']['known_pages'],'current' => $this->_tpl_vars['pagerData']['current_page']), $this);?>


		<div class="search-user-results">
			<?php if ($this->_tpl_vars['users']): ?>
				<?php if (count($_from = (array)$this->_tpl_vars['users'])):
    foreach ($_from as $this->_tpl_vars['user']):
?>
					<?php $this->assign('profile', $this->_tpl_vars['user']->getProfile()); ?>
					<div class="item">
						<div class="screen-name">
							<?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['user'],'image' => 'true'), $this);?>

							<?php if ($this->_tpl_vars['profile']->getRealName() != ''): ?> (real name: <?php echo ((is_array($_tmp=$this->_tpl_vars['profile']->getRealName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
)<?php endif; ?>
						</div>
						<?php if ($this->_tpl_vars['profile']->getAbout() != ''): ?>
							<div class="about">
								<?php echo ((is_array($_tmp=$this->_tpl_vars['profile']->getAbout())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>

							</div>
						<?php endif; ?>
					</div>
				<?php endforeach; endif; unset($_from); ?>
			<?php else: ?>
				<?php if ($this->_tpl_vars['query'] && $this->_tpl_vars['query'] != ''): ?>
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Sorry, no results found for your query.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
				<?php endif; ?>
			<?php endif; ?>
		</div>

		<?php if ($this->_tpl_vars['countResults'] > 7): ?>
			<?php echo smarty_function_pager(array('url' => $this->_smarty_vars['capture']['destUrl'],'total' => $this->_tpl_vars['pagerData']['total_pages'],'known' => $this->_tpl_vars['pagerData']['known_pages'],'current' => $this->_tpl_vars['pagerData']['current_page']), $this);?>


		<?php endif; ?>
	<?php endif; ?>

</div>