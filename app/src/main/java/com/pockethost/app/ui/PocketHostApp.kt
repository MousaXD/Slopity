package com.pockethost.app.ui

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.lifecycle.viewmodel.compose.viewModel
import com.pockethost.app.AppContainer
import com.pockethost.app.domain.DeviceCapabilities
import com.pockethost.app.domain.HostingPlan
import com.pockethost.app.domain.ServerProfile
import com.pockethost.app.service.HostServiceController
import com.pockethost.app.ui.theme.PocketHostTheme

@Composable
fun PocketHostApp(container: AppContainer) {
    val viewModel: DashboardViewModel = viewModel(factory = DashboardViewModel.factory(container))
    val state = viewModel.uiState
    val context = LocalContext.current
    var message by remember { mutableStateOf("Runtime adapters are not installed yet.") }

    val notificationPermissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
    ) { granted ->
        if (granted) {
            HostServiceController.start(context)
            message = "Host foreground service started. No server processes were launched."
        } else {
            message = "Notification permission was denied, so the host service was not started."
        }
    }

    fun requestHostServiceStart() {
        val needsPermission = Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.POST_NOTIFICATIONS,
            ) != PackageManager.PERMISSION_GRANTED

        if (needsPermission) {
            notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        } else {
            HostServiceController.start(context)
            message = "Host foreground service started. No server processes were launched."
        }
    }

    PocketHostTheme {
        DashboardScreen(
            state = state,
            message = message,
            onRefresh = viewModel::refresh,
            onStartHost = ::requestHostServiceStart,
            onStopHost = {
                HostServiceController.stop(context)
                message = "Host foreground service stop requested."
            },
            onPreflight = { profile ->
                message = viewModel.preflight(profile)
            },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DashboardScreen(
    state: DashboardUiState,
    message: String,
    onRefresh: () -> Unit,
    onStartHost: () -> Unit,
    onStopHost: () -> Unit,
    onPreflight: (ServerProfile) -> Unit,
) {
    Scaffold(
        topBar = {
            TopAppBar(title = { Text("PocketHost") })
        },
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            item {
                IntroCard(message = message)
            }
            item {
                CapabilityCard(
                    capabilities = state.capabilities,
                    plan = state.plan,
                    onRefresh = onRefresh,
                )
            }
            item {
                HostControls(onStartHost = onStartHost, onStopHost = onStopHost)
            }
            item {
                Text(
                    text = "Server profiles",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                )
            }
            items(state.profiles, key = ServerProfile::id) { profile ->
                ServerProfileCard(profile = profile, onPreflight = { onPreflight(profile) })
            }
            item {
                Spacer(Modifier.height(24.dp))
                Text(
                    text = "Foundation build: profiles and Android lifecycle exist; executable engines do not.",
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}

@Composable
private fun IntroCard(message: String) {
    Card(colors = CardDefaults.cardColors()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text("A control plane, not a pretend server", fontWeight = FontWeight.Bold)
            Text(
                "PocketHost can model several server families and reserve resources. " +
                    "Each real runtime will arrive behind a tested adapter.",
            )
            HorizontalDivider()
            Text(message, style = MaterialTheme.typography.bodyMedium)
        }
    }
}

@Composable
private fun CapabilityCard(
    capabilities: DeviceCapabilities,
    plan: HostingPlan,
    onRefresh: () -> Unit,
) {
    Card {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text("Device capability", fontWeight = FontWeight.Bold)
                OutlinedButton(onClick = onRefresh) { Text("Refresh") }
            }
            Text("Tier: ${plan.tier}")
            Text("RAM: ${capabilities.availableMemoryMb} MB available / ${capabilities.totalMemoryMb} MB total")
            Text("CPU: ${capabilities.cpuCores} logical cores")
            Text("ABI: ${capabilities.supportedAbis.joinToString()}")
            Text("App storage free: ${capabilities.freeStorageMb} MB")
            Text("Thermal status: ${capabilities.thermalStatus}")
            HorizontalDivider()
            Text("Reserved for Android: ${plan.reservedForAndroidMb} MB")
            Text("Conservative server budget: ${plan.usableForServersMb} MB")
            Text("Recommended concurrent sessions: ${plan.recommendedMaxConcurrentServers}")
            if (plan.recommendedMemoryPerServerMb > 0) {
                Text("Approximate per-session budget: ${plan.recommendedMemoryPerServerMb} MB")
            }
            plan.warnings.forEach { warning ->
                Text("Warning: $warning", style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
private fun HostControls(onStartHost: () -> Unit, onStopHost: () -> Unit) {
    Card {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text("Android host envelope", fontWeight = FontWeight.Bold)
            Text(
                "This starts only the visible foreground service. Runtime sessions remain unavailable.",
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = onStartHost) { Text("Arm host") }
                OutlinedButton(onClick = onStopHost) { Text("Stop host") }
            }
        }
    }
}

@Composable
private fun ServerProfileCard(profile: ServerProfile, onPreflight: () -> Unit) {
    Card {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(profile.displayName, fontWeight = FontWeight.Bold)
            Text("Runtime: ${profile.runtime}")
            Text("Memory request: ${profile.memoryMb} MB")
            Text("Ports: ${profile.ports.joinToString()}")
            Text(profile.description, style = MaterialTheme.typography.bodySmall)
            OutlinedButton(onClick = onPreflight) {
                Text("Run preflight")
            }
        }
    }
}
