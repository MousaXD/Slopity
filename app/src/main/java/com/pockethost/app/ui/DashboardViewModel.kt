package com.pockethost.app.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.pockethost.app.AppContainer
import com.pockethost.app.domain.CapabilityAdvisor
import com.pockethost.app.domain.DeviceCapabilities
import com.pockethost.app.domain.HostingPlan
import com.pockethost.app.domain.ServerProfile

class DashboardViewModel(private val container: AppContainer) : ViewModel() {
    var uiState by mutableStateOf(loadState())
        private set

    fun refresh() {
        uiState = loadState()
    }

    fun preflight(profile: ServerProfile): String =
        container.orchestrator.preflight(profile).summary

    private fun loadState(): DashboardUiState {
        val capabilities = container.capabilityProbe.read()
        return DashboardUiState(
            capabilities = capabilities,
            plan = CapabilityAdvisor.advise(capabilities),
            profiles = container.profileRepository.list(),
        )
    }

    companion object {
        fun factory(container: AppContainer): ViewModelProvider.Factory = viewModelFactory {
            initializer {
                DashboardViewModel(container)
            }
        }
    }
}

data class DashboardUiState(
    val capabilities: DeviceCapabilities,
    val plan: HostingPlan,
    val profiles: List<ServerProfile>,
)
