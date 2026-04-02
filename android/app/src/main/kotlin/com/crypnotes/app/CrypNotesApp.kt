package com.crypnotes.app

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.crypnotes.feature.labels.LabelsDestination
import com.crypnotes.feature.notes.NotesDestination
import com.crypnotes.feature.reminders.RemindersDestination
import com.crypnotes.feature.settings.SettingsDestination
import com.crypnotes.feature.vault.VaultDestination

@Composable
fun CrypNotesApp() {
    val navController = rememberNavController()
    val destinations = listOf(
        TopLevelDestination(NotesDestination.route, NotesDestination.label),
        TopLevelDestination(LabelsDestination.route, LabelsDestination.label),
        TopLevelDestination(RemindersDestination.route, RemindersDestination.label),
        TopLevelDestination(VaultDestination.route, VaultDestination.label),
        TopLevelDestination(SettingsDestination.route, SettingsDestination.label),
    )
    val currentEntry by navController.currentBackStackEntryAsState()
    val currentDestination = currentEntry?.destination

    MaterialTheme {
        Scaffold(
            bottomBar = {
                NavigationBar {
                    destinations.forEach { destination ->
                        NavigationBarItem(
                            selected = currentDestination
                                ?.hierarchy
                                ?.any { it.route == destination.route } == true,
                            onClick = {
                                navController.navigate(destination.route) {
                                    popUpTo(navController.graph.startDestinationId) {
                                        saveState = true
                                    }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            },
                            label = { Text(text = destination.label) },
                            icon = {}
                        )
                    }
                }
            }
        ) { innerPadding ->
            NavHost(
                navController = navController,
                startDestination = NotesDestination.route,
                modifier = Modifier.padding(innerPadding),
            ) {
                composable(NotesDestination.route) {
                    FeaturePlaceholder(NotesDestination.label)
                }
                composable(LabelsDestination.route) {
                    FeaturePlaceholder(LabelsDestination.label)
                }
                composable(RemindersDestination.route) {
                    FeaturePlaceholder(RemindersDestination.label)
                }
                composable(VaultDestination.route) {
                    FeaturePlaceholder(VaultDestination.label)
                }
                composable(SettingsDestination.route) {
                    FeaturePlaceholder(SettingsDestination.label)
                }
            }
        }
    }
}

@Composable
private fun FeaturePlaceholder(label: String) {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Text(text = "$label module is wired")
    }
}

private data class TopLevelDestination(
    val route: String,
    val label: String,
)
